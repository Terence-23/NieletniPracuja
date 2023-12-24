create type contract as enum ('praca', 'dzielo', 'zlecenie', 'tmp');
create type job_hours as enum ('weekend', 'holiday', 'week', 'elastic');
create type job_mode as enum('stationary', 'home', 'hybrid', 'mobile');



create table companies(
    userid UUID primary key not null,
    login varchar(50) unique not null,
    email varchar(255) not null,
    password varchar(255) not null,
    NIP Int not null,
    company_name varchar(255) not null,
    full_name varchar(255) not null,
);

create table users(
    userid uuid primary key not null,
    login varchar(50) unique not null,
    password varchar(255) not null,
    email varchar(255) not null,
    full_name varchar(255) not null,

);

create table jobs(
    jobid serial primary key,
    owner uuid not null,
    creation_time time not null,
    job_location varchar(255),
    contract_type contract not null,
    mode job_mode not null,
    hours job_hours not null,
    description text,
    foreign key (owner)
        references companies(userid) 

)