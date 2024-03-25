generate-migration:
	@ if [ -z "${n}" ]; then (echo 'Missing n=MIGRATION_NAME. e.g. n=create_post_table' >&2 && exit 1) fi
	
	@ sea-orm-cli migrate generate $(n)

up:
	@ sea-orm-cli migrate up

entity:
	@ sea-orm-cli generate entity -o ./src/active_models

refresh:
	@ sea-orm-cli migrate refresh

fresh:
	@ sea-orm-cli migrate fresh