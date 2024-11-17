"use client";

import React from "react";
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
import { Spinner } from "@nextui-org/spinner";
import { PlusCircleIcon, TrashIcon } from "@heroicons/react/24/outline";
import { useAsyncList } from "@react-stately/data";

import transactionsData from "@/transactions.json";

const prettyPrint = (value: any): string => {
  if (value instanceof Date) {
    return value.toISOString().split("T")[0];
  } else if (typeof value === "number") {
    return value.toLocaleString();
  } else if (typeof value === "string") {
    return value.trim();
  } else if (value === null || value === undefined) {
    return "-";
  }

  return value.toString();
};

export default function ExpensesPage() {
  const [page, setPage] = React.useState(1);
  const [isLoading, setLoading] = React.useState(true);

  const PAGE_SIZE = 10;
  const numPages = Math.ceil(transactionsData.length / PAGE_SIZE);
  const hasMore = page < numPages;

  const transactions = transactionsData
    .map((transaction, key) => ({
      key,
      ...transaction,
      date: new Date(transaction.date),
    }))
    .sort((a, b) => a.date.getTime() - b.date.getTime());

  const columns = Object.keys(transactionsData[0]).map((k) => ({
    key: k,
    label: k.toUpperCase(),
  }));

  const transactionsList = useAsyncList<Object, number>({
    async load({ cursor }) {
      setLoading(true);
      const start = cursor ?? 0;
      const items = transactions.reverse().slice(start, start + PAGE_SIZE);
      cursor = start + PAGE_SIZE;
      setPage((prev) => prev + 1);
      setLoading(false);
      return { items, cursor };
    },
  });


  const createNewTransaction = () => {
    const length = transactions.length
    const newTransaction = transactions[length - 1];
    Object.keys(newTransaction).forEach(k => newTransaction[k] = "");
    newTransaction.key = length;
    newTransaction.date = transactions[0].date;
    transactions.push(newTransaction);
    transactionsList.items = transactions.reverse().slice(start, start + PAGE_SIZE);
  }

  return (
    <div className="flex flex-col h-[94vh]">
      <div className="flex items-center justify-end gap-2 py-2">
        <Button
          className="text-sm font-normal text-default-600 bg-default-100"
          startContent={<PlusCircleIcon />}
        />
        <Button
          className="text-sm font-normal text-default-600 bg-default-100"
          startContent={<TrashIcon />}
        />
      </div>
      <Table
        isStriped
        isHeaderSticky
        aria-label="Expenses table"
        color="success"
        defaultSelectedKeys={["1"]}
        selectionMode="single"
        bottomContent={
          hasMore && !isLoading ? (
            <div className="flex w-full items-center justify-between">
              <div />
              <Button
                isDisabled={transactionsList.isLoading}
                variant="flat"
                onPress={transactionsList.loadMore}
              >
                {transactionsList.isLoading && (
                  <Spinner color="white" size="sm" />
                )}
                Load More
              </Button>
              <div>
                <h1>{`${Math.min(transactionsData.length, (page - 1) * PAGE_SIZE)}/${transactionsData.length}`}</h1>
              </div>
            </div>
          ) : null
        }
        className="overflow-scroll"
      >
        <TableHeader columns={columns}>
          {(column) => (
            <TableColumn className="bg-sky-100 text-sky-600" key={column.key}>
              {column.label}
            </TableColumn>
          )}
        </TableHeader>
        <TableBody
          isLoading={isLoading}
          items={transactionsList.items}
          loadingContent={<Spinner label="loading..." />}
        >
          {(item) => (
            <TableRow key={item.key}>
              {(columnKey) => (
                <TableCell>
                  {prettyPrint(getKeyValue(item, columnKey))}
                </TableCell>
              )}
            </TableRow>
          )}
        </TableBody>
      </Table>
    </div>
  );
}
