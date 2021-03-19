#![feature(arc_new_cyclic)]
#![feature(test)]
#![feature(negative_impls)]
#![feature(auto_traits)]
#![feature(specialization)]

extern crate fuzzcheck_mutators;

mod empty_structs;
mod one_field_structs;
mod structs_with_generic_type_params;
mod two_field_structs;

mod enums_with_generic_type_params;
mod enums_with_items_with_and_without_fields;
mod enums_with_multiple_empty_items;
mod enums_with_one_empty_item;
mod enums_with_one_item_multiple_fields;
mod enums_with_one_item_one_field;

mod recursive_enum;
mod recursive_struct;
