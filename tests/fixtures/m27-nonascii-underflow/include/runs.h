#ifndef M27_NONASCII_UNDERFLOW_RUNS_H
#define M27_NONASCII_UNDERFLOW_RUNS_H

/* M27 T0b specimen header: trivial and pure-ASCII; both specimen TUs
 * include it so each TU copy carries one non-ASCII run of its own. */
int m27_runs_probe(int value);

#endif /* M27_NONASCII_UNDERFLOW_RUNS_H */
