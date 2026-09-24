#include "thing.h"

int main() {
    ThingConfig cfg{42, "test_consumer"};
    Thing t(cfg);
    return t.get_id() == 42 ? 0 : 1;
}
