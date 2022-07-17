# Movie Bot
This example focuses on showing how to apply attributes such as choices and min/max values to command options. It also shows how to retrieve data from a sqlite database when prompted by a command. The bot has two commands, one for recommending a movie based on some parameters, and another to set a channel to post about new releases in.

The second command doesn't actually do anything, but you could save the releases channel for each server to a database. If you then poll IMDB for new releases on a timer, you could post them to the saved channel. For this example, we instead just use a static sqlite database.

The example requires a sqlite database containing movie data from imdb - you can generate this using [imdb-sqlite](https://github.com/jojje/imdb-sqlite). Note that this will take up ~10GB of space on your harddrive.