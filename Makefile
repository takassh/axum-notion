generate-migration:
	@ if [ -z "${n}" ]; then (echo 'Missing n=MIGRATION_NAME. e.g. n=create_post_table' >&2 && exit 1) fi
	
	@ sea-orm-cli migrate generate $(n) -d ./libs/repository/migration

up:
	@ sea-orm-cli migrate up -d ./libs/repository/migration

entity:
	@ sea-orm-cli generate entity -o ./libs/repository/src/active_models

refresh:
	@ sea-orm-cli migrate refresh -d ./libs/repository/migration

fresh:
	@ sea-orm-cli migrate fresh -d ./libs/repository/migration

down:
	@ if [ -z "${n}" ]; then (echo 'Missing n=NUM. e.g. n=1' >&2 && exit 1) fi
	@ sea-orm-cli migrate down -n $(n) -d ./libs/repository/migration