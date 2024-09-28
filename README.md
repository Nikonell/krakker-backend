## Setup
Create .env file like this:
```bash
UNISENDER_API_KEY="mykey"
UNISENDER_SENDER_NAME="mysender"
UNISENDER_SENDER_EMAIL="me@unisender.org"
GITHUB_APP_ID="your_app_id"
GITHUB_APP_PRIVATE_KEY="your_app_private_key"
JWT_SECRET="16+random_chars"
DATABASE_URL="postgres://myuser:mypass@localhost/mydb?connection_limit=90&pool_timeout=2"
```
Before the first run, please execute these commands in you terminal:
```bash
source .env
cargo prisma migrate dev # for dev environment
```
Now you can just run the backend application. 
