# TODO

* fix apparent bug where half a hex disappears.
  * repro: match (and therefore remove) the first two blue-outline and blue/green interior hexes which are diagonally downward from the upper left most hexes (non-counting the None apron).
   The bug is that one half of the upper-left most hex appears to have disappeared after the hexes fell down.

* make hexes fall to fill space left by missing hexes
  * if we must have weirdness around the edges, we should render something on the edges to indicate that.
      * for example, the right side halves on the top row cannot fall.
  * add clacking sound when things fall


* should we allow switching hex with empty space to try and make chain reactions?
  * Yeah, probably.

* generate solvable puzzles where the goal is to eliminate all the hexes

* try more mechanics
  * meta
    * how hard is it to make all of these independently toggleable?
      * at runtime?
  * large chunks falling together
  * hexagons with multiple colours that match if any of them match
  * hexagons that drop more hexs on top when they match
    * make all of them like that and make the goal top fill the grid? To `x`% full?
  * special hexes that remove all the ones of that colour when matched
  * scramble the colours within a radius when certain hexes match
  * only allow switching groups of hexes at once. i.e. your cursor is larger than a half hex.
    * maybe it's even disconnected from itself?
