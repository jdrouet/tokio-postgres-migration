CREATE TABLE pets (
	id SERIAL PRIMARY KEY,
	owner_id SERIAL NOT NULL REFERENCES users(id),
	name TEXT UNIQUE NOT NULL
);
