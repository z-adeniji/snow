# Snow - A Tauri Powered UI for DeepSeek with Ollama

**Snow** is a lightweight desktop application built with Tauri, Sveltekit & DaisyUI that provides a user-friendly interface for interacting with DeepSeek when running locally using Ollama.

---

## Features

- **Local AI Interaction**: Seamlessly interact with DeepSeek running locally via Ollama.
- **Simple and Intuitive UI**: A clean and minimalistic interface designed for ease of use.
- **Cross-Platform**: Built with Tauri, Snow works on Windows, macOS, and Linux.
- **Lightweight**: Low resource usage, ensuring smooth performance even on older machines.
- **Customizable**: Easily configure settings to tailor the experience to your needs.

---

## Prerequisites

Before using Snow, ensure you have the following installed:

1. **Ollama**: Snow relies on Ollama to run DeepSeek locally. Follow the [Ollama installation guide](https://ollama.ai/docs) to set it up.
2. **DeepSeek Model**: Install your desired DeepSeek model using Ollama:
   ```bash
   ollama run deepseek-r1:7b
   ```

---

## Installation

### Prebuilt Binaries

Download the latest release for your operating system from the [Releases page](https://github.com/your-repo/snow/releases).

### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/your-repo/snow.git
   cd snow
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Build the application:
   ```bash
   npm run tauri build
   ```
4. The built application will be located in the `src-tauri/target/release` directory.

---

## Usage

1. Launch Snow after installation.
2. Ensure Ollama is running and the DeepSeek model is loaded.
3. Use the input field to send queries to DeepSeek.
4. View responses in the output window.

---

## Configuration

Snow can be configured via the `settings.json` file located in the application's configuration directory. Options include:

- **Ollama Endpoint**: Set the URL for the Ollama server (default: `http://localhost:11434`).
- **Model Name**: Specify the model to use (default: `deepseek`).

---

## Contributing

Contributions are welcome! If you'd like to contribute to Snow, please follow these steps:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Submit a pull request with a detailed description of your changes.

---

## License

Snow is licensed under the MIT License. See the [LICENSE](https://mit-license.org) file for more details.

---

## Support

If you encounter any issues or have questions, please [open an issue](https://github.com/your-repo/snow/issues) on GitHub.

---

Enjoy using Snow to explore the capabilities of DeepSeek locally! ❄️