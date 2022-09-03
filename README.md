# new_tube
This software is used to fetch new videos from channels I am interested in and send them to me via telegram. 

new_tube is not designed for public use and will have frequent breaking changes.

## But why not use subscriptions and notifications from youtube?
Privacy. Of course, this stands in contrast with using services like youtube and telegram, but this is just the beginning. In the future I want to use a scraper or youtube-dl instead of the youtube API.

## How it works
new_tube uses a SQLite database to store video data. The data is retrieved by the youtube data api using the list and video endpoints. New videos can be fetched manually, but a telegram bot exists to do this periodically. Only one allowed user can access the bot, which is determined by an environment variable.

## TODOs
- replace API usage with a scraper approach
- add support for streams and premiers
- add current cli commands as bot commands
- maybe multiuser support
