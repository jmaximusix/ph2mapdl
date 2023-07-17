# PH2 Map Manager for Dedicated Servers

A small interactive cli / tui based app to download and manage Custom Maps from the Steam Workshop on dedicated Perfect Heist 2 servers.

#### Prerequisites
- You have `steamcmd` installed
- You have the Perfect Heist 2 Server installed

### Quick start

- Download the release version for 64-bit Linux or compile it yourself from the rust source code.
- Create a `.env` file in the same directory with the following two lines:
    1. **STEAM_API_KEY**=_your_steam_api_key_.
    You will need a Steam api key to access the public steam API. Get yours [here](https://steamcommunity.com/login/home/?goto=%2Fdev%2Fapikey)!
    2. **INSTALLDIR**=/path/to/install/directory
    The directory of your Perfect Heist Server installation. It has to be the same in order to install the custom maps for the server.
- Run the app in the terminal and enter the name of the Map you want to search in the Search field. Select the Map from the search results using arrow keys and hit `Enter` to select.
- Done! The map should automatically be downloaded and selected on the Server. Make sure to launch the server executable with `CityCustomMap` as the map option.

## Usage

You can search for Custom Maps in the steam workshop. Enter a search query in the search field. Once you hit `Enter` you'll see the first entry in the following format:

```
TigerMansion by TigerMafia
----------------------------------------
"Hotel Kampala - also known as the TigerMansion."
Small/Medium: 5 players (also other sizes).
3 Floors + Underground. Short Ways. 3 Vaults (two small, one big) Big Garden, Small kitchen. Many
options for destruction and drill. Secret ways the robbers can make good use of.
----------------------------------------
📥 2251, ⭐ 26
```

It displays the map name, the map author as well as a shortened description.
The Map name acts as a Hyperlink to the workshop page as well, you should be able to `Ctrl+Click` it in the Terminal.
At the bottom line you see the number of downloads (📥) and "Favourite" ratings (⭐) this map has.

If you already have the map installed on your server it will also say `[installed]` in the top bar.
```
TigerMansion by TigerMafia [installed]
```
Therefore you can also use this script to just change maps without having to ever remember Workshop IDs and manually change the settings file for it again.

By default, each request will retrieve up to 5 search results. You can customize this limit using the `--page-size` (short `-p`) commandline argument. Keep in mind though, that larger numbers may increase the API response time.

You can navigate the results using the arrow keys ($\leftarrow \uparrow \downarrow \rightarrow$) on your keyboard and select a map by pressing `Enter` or `Spacebar`. This will download the map using `steamcmd` in case it isn't installed yet and afterwards select it by modifying the server settings.
 If you want to cancel the map selection, hit `Esc` or `q`.
You'll then be provided with 3 Options:
* **Quit** (Default) Ends the script. You can also press `Ctrl + C` at any time
* **Load more results** This will load the next page of results for the same search query.
* **Modify search** This allows you to edit the search query instead and get new search results for it.

Again, you can navigate these using the up and down arrow keys ($\uparrow \downarrow$) and hit `Enter` or `Spacebar` to select.

##### Additional features:
- You can skip the initial search query by directly typing it in the commandline when launching the script (e.g. `./mapmgr Tiger Mafia`)
- In case you want to reinstall a map, you can use the `--update` commandline option. This will re-download the map you selected.
- If you want to search for a map but don't want to accidentally install anything or change your configuration you can use the `--no-action` commandline argument which will display the map you selected but don't take any further action. However you can always just hit `Ctrl+C` as well so this is more in case you're paranoid or for some reason want to find out the map id, which only gets displayed after selecting it at the moment (apart from being part of hyperlink).