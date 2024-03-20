refresh:
	@ if [ -z "${u}" ]; then (echo 'Missing connection info. e.g. u=mysql://root:password@localhost:3306/bakeries_db' >&2 && exit 1) fi
	
	@ DATABASE_URL="${u}" sea-orm-cli migrate -d . refresh

fresh:
	@ if [ -z "${u}" ]; then (echo 'Missing connection info. e.g. u=mysql://root:password@localhost:3306/bakeries_db' >&2 && exit 1) fi
	
	@ DATABASE_URL="${u}" sea-orm-cli migrate -d . fresh