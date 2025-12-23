[![Version](https://img.shields.io/badge/version-0.1.0-blue.svg)](https://github.com/boiln/myra/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Windows Support](https://img.shields.io/badge/Windows-10%2F11-brightgreen)](https://www.microsoft.com/windows)

> ⚠️ **Disclaimer**: This tool is for educational purposes only. Users are responsible for complying with all applicable laws and terms of service.

# Myra

Powerful network condition simulator that gives you precise control over your connection behavior.

## Prerequisites

Before running Myra, ensure you have the following installed:

**Administrator Privileges** - Required for network packet capture

    - Myra needs to be run as administrator to use WinDivert for packet interception
    - Right-click the app and select "Run as administrator"

## Download

### From Release

Download the latest release from the [releases page](https://github.com/boiln/myra/releases)

### Building from Source

1. Install prerequisites:

    - [Node.js](https://nodejs.org/) (v18 or later)
    - [Rust](https://rustup.rs/) (latest stable)
    - [pnpm](https://pnpm.io/) (latest version)

2. Clone the repository:

    ```bash
    git clone https://github.com/boiln/myra.git
    cd myra
    ```

3. Install dependencies:

    ```bash
    pnpm install
    ```

4. Build the application:
    ```bash
    pnpm tauri build
    ```

## Development

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) with extensions:
    - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
    - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
    - [TypeScript and JavaScript](https://marketplace.visualstudio.com/items?itemName=ms-vscode.vscode-typescript-javascript)

### Development Commands

```bash
# Start development server
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Troubleshooting

### Common Issues

1. "DLL not found" errors:

    - Verify you're running as administrator (right-click -> Run as administrator)
    - Check that WinDivert.dll exists in the application directory
    - Check that WinDivert64.sys exists in the application directory

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Tauri](https://tauri.app/) for the amazing framework
- [WinDivert](https://reqrypt.org/windivert.html) for network capture and manipulation capabilities
- All contributors and users of this project
