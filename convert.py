import pandas
import json

# Read excel document
excel_data_df = pandas.read_excel('Dataset.xlsx', sheet_name='Reservations')

# Convert excel to string
# (define orientation of document in this case from up to down)
# 'split', 'records', 'index', 'table'
thisisjson = excel_data_df.to_json(orient='records')

# Make the string into a list to be able to input in to a JSON-file
thisisjson_dict = json.loads(thisisjson)

# Define file to write to and 'w' for write option -> json.dump()
# defining the list to write from and file to write to
with open('reservations.json', 'w') as json_file:
    json.dump(thisisjson_dict, json_file)
