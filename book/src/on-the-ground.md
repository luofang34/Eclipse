# Standing on the ground

A map says where the shadow goes. **Stand at greatest eclipse** shows what it looks like from
there. The camera flies down from the globe over six seconds and lands facing the Sun, with the
sky darkening as it comes.

<figure class="eclipse" data-query="date=2027-08-02&stand=greatest" data-title="Greatest eclipse of 2027 August 2, from the ground"></figure>

This example flies by itself after two seconds of showing the whole eclipse. A press or scroll
during those two seconds stays on the globe.

**Stand under the Moon** does the same for a lunar eclipse, landing where the Moon is overhead
at greatest eclipse. **See the sky from this place** lands at the place chosen, at its own
maximum.

## What is drawn

The Sun comes from Navigate. So does the Moon, seen from the place itself and not from the
Earth's centre. At the Moon's distance that difference reaches about a degree, more than the
Moon's own width, and it is why an eclipse is seen only along a path at all.

The stars are drawn down to magnitude 6, about what the eye sees in a dark sky, and are placed
again as the clock moves. The MapLibre fork's sky draws all of them.

The sky's brightness follows the share of the Sun left uncovered at the place. It reaches its
full depth when the camera lands, and it changes as the time bar moves.

## Looking around

Drag to look away from the Sun, which stops the view following it, and tick **Keep the Sun in
view** on the full page to return. Scroll to change the field of view, down to half a degree,
at which the Sun fills the frame.

Play the time bar to watch the Moon cross the Sun, at five minutes of eclipse to each second.
