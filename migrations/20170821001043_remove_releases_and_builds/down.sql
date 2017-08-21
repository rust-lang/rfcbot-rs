create table release
(
	id serial not null
		constraint release_pkey
			primary key,
	date date not null
		constraint release_date_key
			unique,
	released boolean not null
);

create table build
(
	id serial not null
		constraint build_pkey
			primary key,
	build_id varchar not null,
	env text not null,
	successful boolean not null,
	message text not null,
	duration_secs integer,
	start_time timestamp,
	end_time timestamp,
	builder_name text not null,
	job_id text default ''::text not null,
	os text not null,
	constraint build_number_builder_name_key
		unique (build_id, env)
);

