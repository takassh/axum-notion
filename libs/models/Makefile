entity:
	@ if [ -z "${u}" ]; then (echo 'Missing connection info. e.g. u=mysql://root:password@localhost:3306/bakeries_db' >&2 && exit 1) fi
	
	@ sea-orm-cli generate entity \
    -u ${u} \
    -o src/entities

refresh:
	@ if [ -z "${u}" ]; then (echo 'Missing connection info. e.g. u=mysql://root:password@localhost:3306/bakeries_db' >&2 && exit 1) fi
	
	@ DATABASE_URL="${u}" sea-orm-cli migrate refresh
	@ sea-orm-cli generate entity \
    -u ${u} \
    -o src/entities

fresh:
	@ if [ -z "${u}" ]; then (echo 'Missing connection info. e.g. u=mysql://root:password@localhost:3306/bakeries_db' >&2 && exit 1) fi
	
	@ DATABASE_URL="${u}" sea-orm-cli migrate fresh
	@ sea-orm-cli generate entity \
    -u ${u} \
    -o src/entities