#include "runs.h"

// M27 T0b E3-shape specimen (PUNCT run): the first character below is
// U+20AC EURO SIGN (UTF-8 bytes E2 82 AC) at column 1, so flush_punct
// subtracts 3 BYTES from the char-counted column+1 (2): DEBUG panics at
// tokenizer.rs:331:21 - the exact E3 site; release wraps silently. The
// guillemet U+00AB (UTF-8 bytes C2 AB) between ASCII operators keeps the
// two-byte punct-run shape (both single-encoded ON PURPOSE - see README).

€uro seam = 1 « 2;
