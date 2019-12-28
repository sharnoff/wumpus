# Wumpus

This is a modified implementation of the original "Hunt the Wumpus" game.
The main differences are that here, mazes are randomly generated and
displayed visually. This visual representation **will not** be topoligically
intuitive. (For example: It may be possible to return to where you started
from by going left twice and up once.)

To try it out, `cargo run -- <number of rooms>` will work.
