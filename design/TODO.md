# TODO

* add clacking sound when things fall

* should we allow switching hex with empty space to try and make chain reactions?
  * Yeah, probably.

* generate solvable puzzles where the goal is to eliminate all the hexes
  * add wall (A hex that doesn't fall)
  * add way to initialize game from a grid
  * start generating small grids of different shapes
  * ensure shapes are connected to the center, but allow non-rectangles
    * what are the further requirements for solvability?

* try more mechanics
  * meta
    * how hard is it to make all of these independently toggleable?
      * at runtime?
  * large chunks falling together
  * hexagons with multiple colours that match if any of them match
  * hexagons that drop more hexes on top when they match
    * make all of them like that and make the goal top fill the grid? To `x`% full?
  * special hexes that remove all the ones of that colour when matched
  * scramble the colours within a radius when certain hexes match
  * only allow switching groups of hexes at once. i.e. your cursor is larger than a half hex.
    * maybe it's even disconnected from itself?
