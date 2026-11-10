import sqlite3
import os

db_path = os.path.expandvars(r"%APPDATA%\com.echo.desktop\ECHO\database\echo.db")
print("Database file exists:", os.path.exists(db_path))

conn = sqlite3.connect(db_path)
c = conn.cursor()

c.execute("PRAGMA user_version;")
version = c.fetchone()[0]
print("Schema user_version:", version)

c.execute("SELECT name FROM sqlite_master WHERE type='table';")
tables = c.fetchall()
print("Tables:", tables)

c.execute("PRAGMA table_info(documents);")
columns = c.fetchall()
print("Columns in documents table:")
for col in columns:
    print(f"  CID {col[0]}: {col[1]} ({col[2]})")

c.execute("SELECT name, sql FROM sqlite_master WHERE type='index';")
indexes = c.fetchall()
print("Indexes on documents:")
for idx in indexes:
    print(f"  Index: {idx[0]}")

