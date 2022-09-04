# new_tube
This software is used to fetch new videos from channels I am interested in and send them to me via telegram. 

new_tube is not designed for public use and will have frequent breaking changes.

## But why not use subscriptions and notifications from YouTube?
Yes, this software seems to be super silly and useless. There are already features built into YouTube itself to let you know what your favorite content creators just uploaded. But I don't want this Big Brother like company to know everything about me. Maybe someone with the same issues finds use in it.

(Also I never save cookies, so I would have to log in every time. Just another me-problem I guess)

## How it works
new_tube uses a SQLite database to store video data. The data is retrieved using [yt_dlp](https://github.com/yt-dlp/yt-dlp) (<3). New videos can be fetched manually, but a telegram bot exists to do this periodically. Only one allowed user can access the bot, which is determined by an environment variable.

## TODOs
- add support for streams and premiers
- add current cli commands as bot commands
- maybe multiuser support
