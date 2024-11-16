"use client";

import {
  Table,
  TableHeader,
  TableColumn,
  TableBody,
  TableRow,
  TableCell,
  getKeyValue,
} from "@nextui-org/table";
import { Button } from "@nextui-org/button";

import { title } from "@/components/primitives";
import transactions from "@/transactions.json";

export default function ExpensesPage() {
  const transactionsWithKey = transactions.map((transaction, key) => ({
    ...transaction,
    date: transaction.date.split("T")[0],
    key,
  }));
  const columns = Object.keys(transactions[0]).map((k) => ({
    key: k,
    label: k.toUpperCase(),
  }));

  return (
    <div className="flex flex-col items-center justify-center gap-4 py-8 md:py-10">
      <h1 className={title()}>Expenses</h1>
      <div className="flex items-center justify-center gap-4 py-4 md:py-6">
        <Button className="text-sm font-normal text-default-600 bg-default-100">
          +
        </Button>
      </div>
      <Table
        isStriped
        color="success"
        selectionMode="single"
        defaultSelectedKeys={["1"]}
        aria-label="Example static collection table"
      >
        <TableHeader columns={columns}>
          {(column) => (
            <TableColumn key={column.key}>{column.label}</TableColumn>
          )}
        </TableHeader>
        <TableBody items={transactionsWithKey.slice(-20).reverse()}>
          {(item) => (
            <TableRow key={item.key}>
              {(columnKey) => (
                <TableCell>{getKeyValue(item, columnKey)}</TableCell>
              )}
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}
