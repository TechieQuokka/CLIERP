#!/usr/bin/env python3
"""
Generate Diesel schema from SQLite database
"""
import sqlite3
import sys

def sql_type_to_diesel_type(sql_type, is_nullable=False):
    """Convert SQLite type to Diesel type"""
    type_mapping = {
        'INTEGER': 'Integer',
        'TEXT': 'Text',
        'REAL': 'Float',
        'FLOAT': 'Float',
        'BOOLEAN': 'Bool',
        'BOOL': 'Bool',
        'DATE': 'Date',
        'TIME': 'Time',
        'DATETIME': 'Timestamp',
        'TIMESTAMP': 'Timestamp',
        'BLOB': 'Binary'
    }

    # Handle some variations
    sql_type = sql_type.upper()
    if 'INT' in sql_type:
        diesel_type = 'Integer'
    elif 'CHAR' in sql_type or 'TEXT' in sql_type:
        diesel_type = 'Text'
    elif 'REAL' in sql_type or 'FLOAT' in sql_type or 'DOUBLE' in sql_type:
        diesel_type = 'Float'
    elif 'DATE' in sql_type:
        if 'TIME' in sql_type:
            diesel_type = 'Timestamp'
        else:
            diesel_type = 'Date'
    else:
        diesel_type = type_mapping.get(sql_type, 'Text')

    if is_nullable:
        return f'Nullable<{diesel_type}>'
    return diesel_type

def generate_schema(db_path):
    """Generate Diesel schema from SQLite database"""
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Get all tables
    cursor.execute("SELECT name FROM sqlite_master WHERE type='table' AND name != 'sqlite_sequence' ORDER BY name")
    tables = cursor.fetchall()

    schema_lines = ['// @generated automatically by generate_schema.py', '']
    joinable_lines = []

    for (table_name,) in tables:
        # Get table info
        cursor.execute(f"PRAGMA table_info({table_name})")
        columns = cursor.fetchall()

        # Get foreign keys
        cursor.execute(f"PRAGMA foreign_key_list({table_name})")
        foreign_keys = cursor.fetchall()

        # Find primary key
        primary_key = None
        for col in columns:
            if col[5]:  # pk column
                primary_key = col[1]
                break

        if not primary_key:
            primary_key = 'id'  # default assumption

        schema_lines.append(f'diesel::table! {{')
        schema_lines.append(f'    {table_name} ({primary_key}) {{')

        for col in columns:
            col_id, col_name, col_type, not_null, default_value, is_pk = col
            is_nullable = not not_null and not is_pk
            diesel_type = sql_type_to_diesel_type(col_type, is_nullable)
            schema_lines.append(f'        {col_name} -> {diesel_type},')

        schema_lines.append('    }')
        schema_lines.append('}')
        schema_lines.append('')

        # Generate joinable statements for foreign keys
        for fk in foreign_keys:
            fk_id, seq, referenced_table, from_col, to_col, on_update, on_delete, match = fk
            if referenced_table != table_name:  # Avoid self-references for now
                joinable_lines.append(f'diesel::joinable!({table_name} -> {referenced_table} ({from_col}));')

    # Add joinable statements
    if joinable_lines:
        schema_lines.extend(joinable_lines)
        schema_lines.append('')

    # Add allow_tables_to_appear_in_same_query
    table_names = [table[0] for table in tables]
    schema_lines.append('diesel::allow_tables_to_appear_in_same_query!(')
    for table_name in table_names:
        schema_lines.append(f'    {table_name},')
    schema_lines.append(');')

    conn.close()
    return '\n'.join(schema_lines)

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: python generate_schema.py <database_path>")
        sys.exit(1)

    db_path = sys.argv[1]
    schema = generate_schema(db_path)
    print(schema)