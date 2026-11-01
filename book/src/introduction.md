# Introduction

[Eclipse Maps](../) draws every solar and lunar eclipse from 2000 BCE to 2999 in the browser.
The eclipses are not stored. They are computed on your machine as you open them, from an
analytic Sun and Moon, in the WebAssembly the page is built from.

This book explains what the maps show and how they are made. The live examples are that same
page in a frame, so anything here can be turned, played and zoomed:

<figure class="eclipse" data-query="date=2027-08-02" data-title="The total solar eclipse of 2027 August 2"></figure>

Each example waits for you to load it, because each one is a full copy of the page.

Drag to turn the globe and scroll to zoom. The bar below the globe plays the eclipse, and
**Greatest** returns to its middle. The full page adds the list of eclipses, the
circumstances of the one on show, and the choice of a place to see it from.

The first part reads the maps. The second is how they are computed. The third is how far to
trust them, since every eclipse is compared with NASA's *Five Millennium Canons*, and the
oldest with what people wrote down at the time.

Years are astronomical, so year 0 is 1 BCE and year -584 is 585 BCE. Dates before 1582
October 15 are in the Julian calendar, as the Canons give them.
