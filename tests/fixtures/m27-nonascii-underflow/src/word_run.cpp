#include "runs.h"

// M27 T0b E3-shape specimen (WORD run): the first word below carries TWO
// U+00E9 characters (UTF-8 bytes C3 A9 each, single-encoded ON PURPOSE -
// see README) and starts at column 1, so flush_word subtracts 11 BYTES
// from a CHAR-counted column of 10: DEBUG panics at tokenizer.rs:311
// (E3 class), release wraps silently. A C3 A9 between ASCII letters and a
// second multi-byte word appear further along the same line (T0b lock).

café_café caféine = café_cup;
