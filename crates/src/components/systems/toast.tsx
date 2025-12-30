import * as React from "react"
import { motion, AnimatePresence } from "framer-motion"
import { Button } from "@/components/ui/button"
import { Card, CardContent } from "@/components/ui/card"
import { Info, Loader } from "lucide-react"

const CheckIcon = () => (
  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 18 18" className="text-green-500">
    <title>circle-check-3</title>
    <g fill="none" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" stroke="currentColor">
      <circle cx="9" cy="9" r="7.25"></circle>
      <path d="M5.5,9c.863,.867,1.537,1.868,2.1,2.962,1.307-2.491,2.94-4.466,4.9-5.923"></path>
    </g>
  </svg>
)

interface ToastProps {
  state: "initial" | "loading" | "success"
  onReset?: () => void
  onSave?: () => void
}

const saveStates = {
  initial: {
    icon: <Info className="w-[18px] h-[18px] text-primary-foreground" />,
    text: "Unsaved changes",
  },
  loading: {
    icon: <Loader className="w-[15px] h-[15px] animate-spin text-primary-foreground" />,
    text: "Saving",
  },
  success: {
    icon: <CheckIcon />,
    text: "Changes Saved",
  },
}

export function Toast({ state: initialState, onReset, onSave }: ToastProps) {
  const [state, setState] = React.useState(initialState)

  React.useEffect(() => {
    if (initialState === "loading") {
      setState("loading")
      const timer = setTimeout(() => {
        setState("success")
        const successTimer = setTimeout(() => {
          setState("initial")
        }, 2000)
        return () => clearTimeout(successTimer)
      }, 3000)
      return () => clearTimeout(timer)
    } else {
      setState(initialState)
    }
  }, [initialState])

  const currentState = saveStates[state]

  const handleSave = () => {
    if (onSave) {
      onSave()
    }
  }

  return (
    <Card className="inline-flex h-10 items-center justify-center gap-4 px-1 py-0 bg-card rounded-[99px] overflow-hidden shadow-lg border">
      <CardContent className="flex items-center p-0">
        <motion.div
          className="inline-flex items-center justify-center gap-2 pl-1.5 pr-3 py-0"
          layout
          transition={{ duration: 0.25, ease: "easeInOut" }}
        >
          <AnimatePresence mode="wait">
            <motion.div
              key={state}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              transition={{ duration: 0.25 }}
            >
              {currentState.icon}
            </motion.div>
          </AnimatePresence>
          <AnimatePresence mode="wait">
            <motion.span
              key={state}
              className="text-card-foreground text-[13px] leading-5 font-normal whitespace-nowrap"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0 }}
            >
              {currentState.text}
            </motion.span>
          </AnimatePresence>
        </motion.div>

        <AnimatePresence>
          {state === "initial" && (
            <motion.div
              className="inline-flex items-center gap-2 pl-0 pr-px py-0"
              initial={{ opacity: 0, width: 0 }}
              animate={{ opacity: 1, width: "auto" }}
              exit={{ opacity: 0, width: 0 }}
              transition={{ duration: 0.25, ease: "easeInOut" }}
            >
              <Button
                variant="ghost"
                className="h-7 px-3 text-[13px] text-card-foreground hover:bg-accent hover:text-accent-foreground rounded-[99px] transition-colors duration-200"
                onClick={onReset}
              >
                Reset
              </Button>
              <Button onClick={handleSave} />
            </motion.div>
          )}
        </AnimatePresence>
      </CardContent>
    </Card>
  )
}
