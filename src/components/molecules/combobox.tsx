import * as React from "react";
import { cn } from "@/lib/utils"
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command"
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover"

import { Button } from "@/components/ui/button"
import { Check, ChevronsUpDown, DivideSquareIcon } from "lucide-react"

export type ComboBoxEntry = {
  value: string;
  logo: React.ElementType;
  label: string;
}

type ComboBoxList = {
  title: string;
  list: Array<ComboBoxEntry>;
}

export function ComboBox({ title, list }: ComboBoxList) {
  const [activeVault, setActiveVault] = React.useState("")
  const [open, setOpen] = React.useState(false)
  const [value, setValue] = React.useState("")

  return (
    <>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="w-[200px] justify-between"
          >
            {activeVault
              ? list.find((list) => list.value === value)?.label
              : `Select ${title}...`}
            <ChevronsUpDown className="opacity-50" />
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
                      setActiveVault(currentValue === activeVault ? "" : currentValue)
                      setOpen(false)
                    }}
                  >
                    {entry.label}
                    <Check
                      className={cn(
                        "ml-auto",
                        activeVault === entry.value ? "opacity-100" : "opacity-0"
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
  )
}
