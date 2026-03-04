# Security Policy

## Supported Versions

| Version | Supported          |
|---------|--------------------|
| 1.5.x   | Yes                |
| < 1.5   | No                 |

## Reporting a Vulnerability

If you discover a security vulnerability, **do not open a public issue**.

Instead, please report it privately by emailing the maintainer or using GitHub's [private vulnerability reporting](https://github.com/krabhi4/cli-player/security/advisories/new).

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Acknowledgment:** Within 48 hours
- **Fix:** Best effort, typically within 7 days for critical issues

## Security Considerations

- **Credentials:** Server passwords are encrypted with Fernet (AES-128-CBC) and stored locally at `~/.config/cli-music-player/config.json`. The encryption key is derived per-installation.
- **Network:** The app communicates with Navidrome servers over HTTP/HTTPS using the Subsonic API. Use HTTPS for remote servers.
- **No telemetry:** The app does not phone home or collect any data.
