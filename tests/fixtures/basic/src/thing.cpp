#include "thing.h"

Thing::Thing(ThingConfig config) : config_(config) {}

int Thing::get_id() const {
    return config_.id;
}
