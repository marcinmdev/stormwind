#### STORMWIND

Weather indicator for [Waybar](https://github.com/Alexays/Waybar/) using [Open-Meteo.com](https://open-meteo.com/)

### Features
* main indicator: weather icon and temperature
* tooltip: wind speed, humidity, cloud coverage

### CLI usage
* `--lat` float, Location latitude (example: 52.52), required 
* `--lon` float, Location longitude (example: 13.41), required
* `--units-temperature` Temperature units, default: `celsius`. Valid values: `celsius`, `fahrenheit`
* `--units-wind-speed` Wind speed units, default: `kmh`. Valid values: `kmh`, `ms`, `mph`, `kn`
* `--units-precipitation` Precipitation units, default: `mm`. Valid values `mm`, `inch`

### Requirements
* [nerd font](https://www.nerdfonts.com/)

### Waybar config

Assuming that `stormwind` is in your path:

```json
"custom/stormwind": {
    "exec": "stormwind --lat xx --lon yy",
    "interval": 600,
    "return-type": "json",
    "exec-if":  "wget -q --spider https://google.com",
    },
```

### FAQ
* Can it be used with Polybar/other status bars?

Yes, `jq` can be used to extract data from output, ie. `stormwind --lat xx --lon yy | jq -r '.text'`

* I would like to have more data displayed in main indicator / tooltip

Please create issue. I'm open to suggestion.

## License
#TODO github url
Waybar is licensed under the MIT license. [See LICENSE for more information](https://github.com/TODO).
