// Mock pg_config.

fn main() {
    println!("BINDIR = /opt/data/pgsql-17.2/bin");
    println!("MANDIR = /opt/data/pgsql-17.2/share/man");
    println!("PGXS = /opt/data/pgsql-17.2/lib/pgxs/src/makefiles/pgxs.mk");
    println!("CFLAGS_SL = ");
    println!("LIBS = -lpgcommon -lpgport -lxml2 -lssl -lcrypto -lz -lreadline -lm ");
    println!("VERSION = PostgreSQL 17.2");
}
