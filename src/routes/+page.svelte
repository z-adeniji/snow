<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { listen, type UnlistenFn } from "@tauri-apps/api/event";
    import { Marked } from "marked";
    import type { Tokens } from "marked";
    import { markedHighlight } from "marked-highlight";
    import hljs from 'highlight.js';
    import { onMount } from "svelte";
    import Database from "@tauri-apps/plugin-sql";

    const renderer = {
        code(token: Tokens.Code): string {
            const decodedCode = new DOMParser().parseFromString(token.text, 'text/html').body.textContent || token.text;
            const language = token.lang && hljs.getLanguage(token.lang) ? token.lang : "plaintext";
            const highlightedCode = hljs.highlight(decodedCode, { language }).value;

            return `
                <div class="relative rounded-lg">
                    <div class="flex justify-between items-center bg-gray-200 dark:bg-gray-700 px-4 py-2 rounded-t-lg">
                        <span class="text-sm text-gray-700 dark:text-gray-300">${language}</span>
                        <button class="copy-button text-sm text-gray-700 dark:text-gray-300 hover:text-gray-900 dark:hover:text-gray-100">
                            Copy
                        </button>
                    </div>
                    <pre class="bg-gray-100 dark:bg-gray-800 p-0 m-0 rounded-b-lg overflow-x-auto whitespace-pre-line">
                        <code class="hljs language-${language} !whitespace-pre">${highlightedCode}</code>
                    </pre>
                </div>
            `;
        },
    };

     const marked = new Marked(
        markedHighlight({
            langPrefix: 'hljs language-',
            highlight(code, lang) {
                const language = hljs.getLanguage(lang) ? lang : 'plaintext';
                return hljs.highlight(code, { language }).value;
            },
        })
    );
    
    marked.setOptions({
        gfm: true,
        breaks: false,
    });

    type Settings = {
        endpoint: string;
        model_name: string;
    };

    type Message = {
        role: "user" | "assistant";
        content: string;
    };

    let conversationHistory: Message[] = $state([]);
    let input = $state("");
    let sending = $state(false);
    let settings: Settings | null = $state(null);
    let accumulatedResponse = "";
    let currentConversationId = $state<number | null>(null);
    let conversations = $state<Array<{id: number, title: string}>>([]);

    marked.use({ renderer });

    function cleanResponse(response: string): string {
        const thinkContent = response.match(/<think>([\s\S]*?)<\/think>/)?.[1] || response.replace(/<think>[\s\S]*?<\/think>/g, '').trim();
        const mainContent = response.match(/<think>([\s\S]*?)<\/think>/)?.[1]? response.replace(/<think>[\s\S]*?<\/think>/g, '').trim(): '';
        
        return `
            <details ${thinkContent.length > 7 ? 'open' : 'close'} class="think-process mt-4">
                <summary class="text-sm cursor-pointer">
                    Show thought process
                </summary>
                <div class="think-content mt-2 p-4 bg-gray-200 dark:bg-gray-700 rounded-lg whitespace-pre-line">
                    ${thinkContent}
                </div>
            </details>
            ${marked.parse(mainContent)}
        `;
    }

    //database////////////////////////////////////////////////////////////////////////////////////////////////////////
    async function saveMessage(message: Message) {
        const db = await Database.load('sqlite:conversations.db');

        if (!currentConversationId) {
            const result = await db.execute(
            'INSERT INTO conversations (title) VALUES (?)',
            ['New Chat']
            );
            if (result.lastInsertId) currentConversationId = result.lastInsertId;
        }
        
        await db.execute(
            'INSERT INTO messages (conversation_id, role, content) VALUES (?, ?, ?)',
            [currentConversationId, message.role, message.content]
        );
    }

    async function loadConversations() {
        const db = await Database.load('sqlite:conversations.db');
        conversations = await db.select<Array<{id: number, title: string}>>(
            'SELECT id, title FROM conversations ORDER BY created_at DESC'
        );
    }

    async function loadConversation(id: number) {
        const db = await Database.load('sqlite:conversations.db');
        const messages = await db.select<Array<Message>>(
            'SELECT role, content FROM messages WHERE conversation_id = ? ORDER BY timestamp ASC',
            [id]
        );
        conversationHistory = messages;
        currentConversationId = id;
    }

    async function updateConversationTitle(conversationId: number, firstMessage: string) {
        const db = await Database.load('sqlite:conversations.db');
        const title = firstMessage.substring(0, 50);
        await db.execute(
            'UPDATE conversations SET title = ? WHERE id = ?',
            [title, conversationId]
        );
        await loadConversations();
    }

    async function deleteConversation(id: number) {
        const db = await Database.load('sqlite:conversations.db');
        
        await db.execute('DELETE FROM messages WHERE conversation_id = ?', [id]);
        await db.execute('DELETE FROM conversations WHERE id = ?', [id]);
        
        if (currentConversationId === id) {
            currentConversationId = null;
            conversationHistory = [];
        }
        
        await loadConversations();
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

    async function handleSubmit() {
        sending = true;
        input = input.trim();
        if (!input) {
            sending = false;
            return
        };

        let thing = "";

        accumulatedResponse = ""

        conversationHistory = [
            ...conversationHistory,
            { role: "user", content: input },
            { role: "assistant", content: "" }
        ];

        const currentConversation = conversationHistory;

        const wasNewConversation = !currentConversationId;
        await saveMessage({ role: "user", content: input });

        if (wasNewConversation && currentConversationId) {
            await updateConversationTitle(currentConversationId, input);
            await loadConversations();
        }
        
        input = "";

        const unlisten = await listen<string>("ollama-chunk", async (event) => {
            accumulatedResponse += event.payload;

            const processed = cleanResponse(accumulatedResponse);

            conversationHistory = conversationHistory.map((msg, index) => {
                if (index === conversationHistory.length - 1) {
                    thing = processed
                    return { ...msg, content: processed};
                }
                return msg;
            });
        });

        try {
            await invoke("query_ollama", { conversation: currentConversation });
        } catch (error) {
            conversationHistory = conversationHistory.map((msg, index) => {
                if (index === conversationHistory.length - 1) {
                    return { ...msg, content: `error:  ${error}. Did you run 'ollama serve' in your terminal?`}
                }
                return msg;
            })
        } finally {
            saveMessage({ role: "assistant", content: thing });
            sending = false;
            unlisten?.();
        }
    }

    onMount(async() => {
        settings = await invoke("get_settings");

        const db = await Database.load('sqlite:conversations.db');
        await db.execute(`
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                conversation_id INTEGER,
                role TEXT CHECK(role IN ('user', 'assistant')),
                content TEXT,
                timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY(conversation_id) REFERENCES conversations(id)
            );
        `);

        await loadConversations()
    })
</script>

<div class="drawer">
    <input id="sidebar" type="checkbox" class="drawer-toggle" />
    <div class="drawer-content min-h-screen bg-white dark:bg-gray-900 h- flex flex-col items-center">
        <header class="w-screen sticky top-0 bg-white dark:bg-gray-900 z-40">
            <div class="navbar">
                <div class="flex-none">
                    <!-- svelte-ignore a11y_consider_explicit_label -->
                    <label for="sidebar" class="btn btn-ghost drawer-button">
                        <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        class="inline-block h-5 w-5 stroke-current">
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M4 6h16M4 12h16M4 18h16"></path>
                        </svg>
                    </label>
                </div>
                <div class="flex-1">
                  <!-- svelte-ignore a11y_missing_attribute -->
                  <a href="https://github.com/z-adeniji/snow" class="btn btn-ghost text-xl">snow</a>
                </div>
            </div>
        </header>
        <!--chat stuff-->
        {#if conversationHistory.length === 0}
            <div class="absolute w-full h-full flex justify-center items-center">
                <div class="hero-content text-center">
                    <div class="w-full">
                        <h1 class="text-5xl font-bold">Hi, I'm {settings?.model_name}.</h1>
                        <p class="py-6">How can I help you?</p>
                        <form onsubmit={handleSubmit} class="flex flex-row w-full">
                            <!-- svelte-ignore element_invalid_self_closing_tag -->
                            <div class="w-full tooltip tooltip-bottom" data-tip="Don't send empty messages">
                                <textarea class="w-full mr-2 resize-none outline-none border-t-none border-none border-b-none" rows="1" bind:value={input} placeholder="Ask DeepSeek..." />
                            </div>
                            <button type="submit" class="btn" disabled={sending}>
                                {sending ? "Sending..." : "Send"}
                            </button>
                        </form>
                    </div>
                </div>
            </div>
        {:else}
            <div class="w-[50%] mb-auto ">
                {#each conversationHistory as convo, index}
                    {#if convo.role === "user"}
                        <div class="chat chat-end">
                            <div class="chat-header mr-3">user</div>
                            <div class="chat-bubble p-6 rounded-xl bg-gray-100 dark:bg-gray-800">{convo.content}</div>
                        </div>
                    {:else if convo.role === "assistant"}
                        <div class="chat chat-start">
                            <div class="chat-header ml-3">{settings?.model_name}</div>
                            <div class="chat-bubble prose-pre:mt-0 prose p-6 rounded-xl bg-gray-100 dark:bg-gray-800">
                                {#if sending}
                                    {#if convo.content.length > 0}
                                        {@html convo.content}
                                    {:else}
                                        {@html '<span class="loading loading-ring loading-xs ml-2"></span>'}
                                    {/if}
                                {:else}
                                    {@html convo.content}
                                {/if}
                            </div>
                        </div>
                    {/if}  
                {/each}
            </div>
            <!--footer-->
            <footer class="p-10 w-screen flex items-center justify-center">
                <form onsubmit={handleSubmit} class="flex flex-row w-[50%]">
                    <!-- svelte-ignore element_invalid_self_closing_tag -->
                    <textarea class="w-full mr-2 resize-none" rows="1" bind:value={input} placeholder="Ask DeepSeek..." />
                    <button type="submit" class="btn" disabled={sending}>
                        {sending ? "Sending..." : "Send"}
                    </button>
                </form>
            </footer>
        {/if}
    </div>
    <div class="drawer-side z-50">
        <label for="sidebar" aria-label="close sidebar" class="drawer-overlay"></label>
        <ul class="menu bg-base-200 text-base-content min-h-full w-80 p-4">
            <h1>snow beta v1.0</h1>
            <button class="btn mt-5" onclick={() => {
                    currentConversationId = null;
                    conversationHistory = []
                }}
            >
                New Conversation
            </button>
            <div class="divider"></div>
            {#each conversations as conversation}
                <li class="group relative hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg">
                    <!-- svelte-ignore a11y_click_events_have_key_events -->
                    <!-- svelte-ignore a11y_missing_attribute -->
                    <!-- svelte-ignore a11y_no_static_element_interactions -->
                    <a onclick={() => loadConversation(conversation.id)} class="cursor-pointer pr-8">
                        {conversation.title}
                    </a>
                    <button 
                        class="absolute right-2 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 btn btn-xs btn-ghost text-error"
                        onclick={() => {

                                deleteConversation(conversation.id)
                            
                        }}
                    >
                        ✕
                    </button>
                </li>
            {/each}
        </ul>
    </div>
</div>