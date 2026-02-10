"""Entry point for the CLI Music Player."""

import argparse
import sys

from . import __version__


def main():
    parser = argparse.ArgumentParser(
        description="CLI Music Player for Navidrome",
        prog="music-player",
    )
    parser.add_argument(
        "-v", "-V", "--version",
        action="version",
        version=f"%(prog)s {__version__}",
    )
    parser.add_argument(
        "--config-dir",
        help="Custom config directory (default: ~/.config/cli-music-player)",
        default=None,
    )
    parser.add_argument(
        "--server",
        help="Server name to connect to initially",
        default=None,
    )

    args = parser.parse_args()

    # Set custom config dir if specified
    if args.config_dir:
        from . import config as cfg
        from pathlib import Path
        cfg.CONFIG_DIR = Path(args.config_dir)
        cfg.CONFIG_FILE = cfg.CONFIG_DIR / "config.json"

    from .app import MusicPlayerApp

    app = MusicPlayerApp()

    # If a specific server was requested, try to select it
    if args.server:
        for i, srv in enumerate(app.config.servers):
            if srv.name.lower() == args.server.lower():
                app.config.set_active_server(i)
                break
        else:
            print(f"Server '{args.server}' not found in config.")
            sys.exit(1)

    app.run()


if __name__ == "__main__":
    main()
