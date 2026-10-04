import rustsheet as rs

wb = rs.Workbook()

# Edits are atomic: they apply on a clean exit and are discarded if the block raises.
with wb.edit() as tx:
    tx["Sheet1!A1"] = 10
    tx["Sheet1!A2"] = rs.Formula("=A1*2")
    tx["Sheet1!A3"] = "=1/0"

print(wb["Sheet1!A2"])  # 20.0
print(wb["Sheet1!A3"])  # #DIV/0!  (an rs.XlError value, not an exception)
print(wb.changes["values"])  # cells whose value changed in the last edit
wb.save_xlsx("facade.xlsx")
