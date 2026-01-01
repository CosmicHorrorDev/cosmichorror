> [!IMPORTANT]
> Moved to codeberg :)
>
> <https://codeberg.org/CosmicHarper/blog>

---

# _cosmichorror_

Home to my personal blog :)

## How's it built?

This is a static site that uses [Hugo](https://gohugo.io) as the SSG with some
local changes to [`chroma`](https://github.com/alecthomas/chroma) to improve the
syntax highlighting. Hopefully all of those will be upstreamed over time because
I don't feel like maintaining this fork forever

## Oooo pretty...

The theme is [Fuji](https://github.com/dsrkafuu/hugo-theme-fuji) with the colors
swapped over to fit [Rosé Pine's](https://rosepinetheme.com) colorscheme among
various other tweaks

I'm not planning on maintaining the theme enough for anyone's use other than my.
Hence why the theme exists inside this repo instead of forking Fuji

## Where's it hosted?

It's currently being hosted in a GCP bucket with a loadbalancer to handle HTTPS.
Currently this is cheap and simple enough that I'm not looking to switch
although I would like to automate deployments instead of having to manually push
from my laptop

I'm really happy with my decision to use a static site at the moment, so it's
unlikely I'll need to manage web-servers in the future

## License (CC BY-SA 4.0)

(C) 2025 Cosmic Harper

This work is licensed under <a href="https://creativecommons.org/licenses/by-sa/4.0/">CC BY-SA 4.0</a><img src="https://mirrors.creativecommons.org/presskit/icons/cc.svg" alt="" style="max-width: 1em;max-height:1em;margin-left: .2em;"><img src="https://mirrors.creativecommons.org/presskit/icons/by.svg" alt="" style="max-width: 1em;max-height:1em;margin-left: .2em;"><img src="https://mirrors.creativecommons.org/presskit/icons/sa.svg" alt="" style="max-width: 1em;max-height:1em;margin-left: .2em;">
