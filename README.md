# Whatstat
A WhatsApp chat analyser command line tool.

## Usage
- Build with the --release option in cargo for a 76x speed increase. 
- Use the 'Email chat' in the 'more' menu of a chat to mail a chat history to yourself. 
Choose 'Without media' because this tool will not analyse any media files.

> NOTE: This program will only correctly parse WhatsApp logs made with an en_US or en_GB locale.

- Run whatstat_cli or whatstat_cli.exe

## TODO
- Emoji support
- Other locale/language support 
- Group logical analyses together
- Plugin support
- Make it possible to call `analyse()` externally
- Command line options to enable/disable certian analyses
- Tidy JSON
