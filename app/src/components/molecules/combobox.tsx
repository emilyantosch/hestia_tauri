import * as React from "react";
import { cn } from "@/lib/utils";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

import { Button } from "@/components/ui/button";
import { Check, ChevronsUpDown } from "lucide-react";

export type ComboBoxEntry = {
  value: string;
  logo: React.ElementType;
  label: string;
};

type ComboBoxList = {
  title: string;
  list: Array<ComboBoxEntry>;
};

export function ComboBox({ title, list }: ComboBoxList) {
  const [open, setOpen] = React.useState(false);
  const [value, setValue] = React.useState("");

  return (
    <>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="w-[200px] justify-between"
          >
            {value
              ? list.find((entry) => entry.value === value)?.label
              : `Select ${title}...`}
            <ChevronsUpDown className="ml-2 h-4 w-4 shrink-0 opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-[200px] p-0">
          <Command>
            <CommandInput placeholder={`Search ${title}...`} className="h-9" />
            <CommandList>
              <CommandEmpty>{`No ${title} found.`}</CommandEmpty>
              <CommandGroup>
                {list.map((entry) => (
                  <CommandItem
                    key={entry.value}
                    value={entry.value}
                    onSelect={(currentValue) => {
                      setValue(currentValue === value ? "" : currentValue);
                      setOpen(false);
                    }}
                  >
                    {entry.label}
                    <Check
                      className={cn(
                        "ml-auto h-4 w-4",
                        value === entry.value ? "opacity-100" : "opacity-0",
                      )}
                    />
                  </CommandItem>
                ))}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </>
  );
}
