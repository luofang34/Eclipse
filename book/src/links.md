# Links to a view

The page's address names what it shows, so a view can be shared by copying it.

| Part | Meaning |
| ---- | ------- |
| `date=YYYY-MM-DD` | Opens the eclipse of that date, by TT, or `-YYYY-MM-DD` before year 0. A date with no eclipse opens another eclipse of that year. |
| `stand=greatest` | With a `date` that names an eclipse, shows the whole eclipse for two seconds, then flies down to stand at greatest eclipse, or under the Moon of a lunar one. A press or scroll in those two seconds stays on the globe. |
| `embed=1` | Leaves out the side panel, for a frame in another page. |

For example:

    https://luofang34.github.io/Eclipse/?date=2027-02-06&stand=greatest

## Framing the page

The examples in this book are frames of the page:

```html
<iframe src="https://luofang34.github.io/Eclipse/?date=2027-08-02&embed=1"
        title="The total solar eclipse of 2027 August 2"></iframe>
```

Each frame downloads the page's WebAssembly, about 5 MB, and takes a GPU context of its own, and
the browser needs WebGPU. Load a frame on request when a page has more than one.
