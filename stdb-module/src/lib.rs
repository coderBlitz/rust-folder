use spacetimedb::{ReducerContext, SpacetimeType, Table};

#[derive(SpacetimeType)]
pub struct Vec2 {
	pub x: f64,
	pub y: f64,
}

#[spacetimedb::table(name = stuff, public)]
pub struct Thing {
	#[primary_key]
	#[auto_inc]
	id: u64,
	name: String,
	one: Vec2,
	two: Vec2,
}

#[spacetimedb::reducer(init)]
pub fn init(_ctx: &ReducerContext) {
	// Called when the module is initially published
}

#[spacetimedb::reducer(client_connected)]
pub fn identity_connected(_ctx: &ReducerContext) {
	// Called everytime a new client connects
}

#[spacetimedb::reducer(client_disconnected)]
pub fn identity_disconnected(_ctx: &ReducerContext) {
	// Called everytime a client disconnects
}

#[spacetimedb::reducer]
pub fn add(ctx: &ReducerContext, name: String, one: Vec2, two: Vec2) {
	ctx.db.stuff().insert(Thing {
		id: 0,
		name: name.to_string(),
		one,
		two,
	});
}

#[spacetimedb::reducer]
pub fn add_batch(ctx: &ReducerContext, items: Vec<Thing>) {
	for item in items {
		ctx.db.stuff().insert(item);
	}
}