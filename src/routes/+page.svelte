<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { marked } from "marked";
    
    let input = $state("");
    let sending = $state(false);
    let totalMessages = $state<{ deepSeek: string; user: string }[]>([]);
    let currentMessageIndex = $state(-1);

    async function handleSubmit() {
        sending = true;
        input = input.trim();
        if (!input) return;

        // Add new message pair
        totalMessages = [...totalMessages, { user: input, deepSeek: "" }];
        currentMessageIndex = totalMessages.length - 1;
        const userInput = input;
        input = ""; // Clear input immediately

        try {
            const result: string = await invoke("query_ollama", { prompt: userInput });
            const response = JSON.parse(result).response;
            
            // Process response
            const formattedResponse = response
                .replace(/<think>[\s\S]*?<\/think>/g, '')
                .trimStart();

            //convert markdown to html
            const htmlResponse = await marked.parse(formattedResponse);

            // Stream response
            let index = 0;
            const updateInterval = 50; // Faster perceived updates
            const chunkSize = 3; // Characters per update

            sending = false;

            const interval = setInterval(() => {
                if (index < htmlResponse.length) {
                    // Update in chunks
                    const chunk = htmlResponse.slice(index, index + chunkSize);
                    totalMessages[currentMessageIndex].deepSeek += chunk;
                    index += chunkSize;
                    
                    // Force array update for reactivity
                    totalMessages = totalMessages;
                } else {
                    clearInterval(interval);
                }
            }, updateInterval);

        } catch (error) {
            console.error("Error:", error);
            sending = false;
        }
    }
</script>

<div class="w-full">
    <h1 class="text-[50px]">snow</h1>

    {#each totalMessages as convo, index}
        <div class="chat chat-end">
            <div class="chat-header">You</div>
            <div class="bg-red-400">{convo.user}</div>
        </div>
        <div class="chat chat-start">
            {#if sending && currentMessageIndex === index}
                <span class="loading loading-ring loading-lg"></span>
            {:else}
                <div class="chat-header">DeepSeek R1</div>
                <div class="bg-red-400">
                    {@html convo.deepSeek}
                </div>
            {/if}
        </div>
    {/each}

    <form onsubmit={handleSubmit}>
        <input class="w-full" bind:value={input} placeholder="Ask DeepSeek..." />
        <button type="submit" disabled={sending}>
            {sending ? "Sending..." : "Send"}
        </button>
    </form>
</div>