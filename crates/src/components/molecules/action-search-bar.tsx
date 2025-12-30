"use client"

import * as React from "react"
import { motion, AnimatePresence, MotionConfig } from "framer-motion"
import { Button } from "@/components/ui/button"
import { cn } from "@/lib/utils"
import { ChevronDown, Shirt, Briefcase, Smartphone, Home, Layers } from "lucide-react"
import { useClickAway } from "@/hooks/use-click-away"

interface Category {
  id: string
  label: string
  icon: React.ElementType
  color: string
}

const categories: Category[] = [
  { id: "all", label: "All", icon: Layers, color: "#A06CD5" },
  { id: "lifestyle", label: "Lifestyle", icon: Shirt, color: "#FF6B6B" },
  { id: "desk", label: "Desk", icon: Briefcase, color: "#4ECDC4" },
  { id: "tech", label: "Tech", icon: Smartphone, color: "#45B7D1" },
  { id: "home", label: "Home", icon: Home, color: "#F9C74F" },
]

const IconWrapper = ({
  icon: Icon,
  isHovered,
  color,
}: { icon: React.ElementType; isHovered: boolean; color: string }) => (
  <motion.div className="w-4 h-4 mr-2 relative" initial={false} animate={isHovered ? { scale: 1.2 } : { scale: 1 }}>
    <Icon className="w-4 h-4" />
    {isHovered && (
      <motion.div
        className="absolute inset-0"
        style={{ color }}
        initial={{ pathLength: 0, opacity: 0 }}
        animate={{ pathLength: 0.1, opacity: 1 }}
        transition={{ duration: 0.1, ease: "easeInOut" }}
      >
        <Icon className="w-4 h-4" strokeWidth={2} />
      </motion.div>
    )}
  </motion.div>
)

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      when: "beforeChildren",
      staggerChildren: 0.1,
    },
  },
}

const itemVariants = {
  hidden: { opacity: 0, y: -10 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.1,
      ease: [0.25, 0.1, 0.25, 1],
    },
  },
}

export default function FluidDropdown() {
  const [isOpen, setIsOpen] = React.useState(false)
  const [selectedCategory, setSelectedCategory] = React.useState<Category>(categories[0]) // Default to "All"
  const [hoveredCategory, setHoveredCategory] = React.useState<string | null>(null)
  const dropdownRef = React.useRef<HTMLDivElement>(null)

  // Handle click outside to close dropdown
  useClickAway(dropdownRef, () => setIsOpen(false))

  // Handle keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      setIsOpen(false)
    }
  }

  return (
    <MotionConfig reducedMotion="user">
      <div className="min-h-screen flex items-center justify-center bg-black">
        <div
          className="w-full px-4 relative"
          style={{ maxWidth: "calc(24rem - 40px)", height: "40px" }}
          ref={dropdownRef}
        >
          <Button
            variant="outline"
            onClick={() => setIsOpen(!isOpen)}
            className={cn(
              "w-full justify-between bg-neutral-900 text-neutral-400 font-mono",
              "hover:bg-neutral-800 hover:text-neutral-200",
              "focus:ring-2 focus:ring-neutral-700 focus:ring-offset-2 focus:ring-offset-black",
              "transition-all duration-100 ease-in-out",
              "border border-transparent focus:border-neutral-700",
              "h-10", // Add this line to set a fixed height
              isOpen && "bg-neutral-800 text-neutral-200",
            )}
            aria-expanded={isOpen}
            aria-haspopup="true"
          >
            <span className="flex items-center">
              <IconWrapper icon={selectedCategory.icon} isHovered={false} color={selectedCategory.color} />
              {selectedCategory.label}
            </span>
            <motion.div
              animate={{ rotate: isOpen ? 180 : 0 }}
              whileHover={{ rotate: isOpen ? 180 : 180 }}
              transition={{ duration: 0.1 }}
              style={{
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                width: "20px",
                height: "20px",
              }}
            >
              <ChevronDown className="w-4 h-4" />
            </motion.div>
          </Button>

          <AnimatePresence>
            {isOpen && (
              <motion.div
                initial={{ opacity: 1, y: 0, height: 0 }}
                animate={{
                  opacity: 1,
                  y: 0,
                  height: "auto",
                  transition: {
                    type: "spring",
                    stiffness: 500,
                    damping: 30,
                    mass: 1,
                  },
                }}
                exit={{
                  opacity: 0,
                  y: 0,
                  height: 0,
                  transition: {
                    type: "spring",
                    stiffness: 500,
                    damping: 30,
                    mass: 1,
                  },
                }}
                className="absolute left-4 right-4 top-full mt-2"
                onKeyDown={handleKeyDown}
              >
                <motion.div
                  className={cn("absolute w-full rounded-lg border border-neutral-800", "bg-neutral-900 p-1 shadow-lg")}
                  initial={{ borderRadius: 8 }}
                  animate={{
                    borderRadius: 12,
                    transition: { duration: 0.1 },
                  }}
                  style={{ transformOrigin: "top" }}
                >
                  <motion.div className="py-2 relative" variants={containerVariants} initial="hidden" animate="visible">
                    <motion.div
                      layoutId="hover-highlight"
                      className="absolute inset-x-1 bg-neutral-800 rounded-md"
                      animate={{
                        y:
                          categories.findIndex((c) => (hoveredCategory || selectedCategory.id) === c.id) * 40 +
                          (categories.findIndex((c) => (hoveredCategory || selectedCategory.id) === c.id) > 0 ? 20 : 0),
                        height: 40,
                      }}
                      transition={{
                        type: "spring",
                        bounce: 0.15,
                        duration: 0.1,
                      }}
                    />
                    {categories.map((category, index) => (
                      <React.Fragment key={category.id}>
                        {index === 1 && (
                          <motion.div className="mx-4 my-2.5 border-t border-neutral-700" variants={itemVariants} />
                        )}
                        <motion.button
                          onClick={() => {
                            setSelectedCategory(category)
                            setIsOpen(false)
                          }}
                          onHoverStart={() => setHoveredCategory(category.id)}
                          onHoverEnd={() => setHoveredCategory(null)}
                          className={cn(
                            "relative flex w-full items-center px-4 py-2.5 text-sm rounded-md",
                            "transition-colors duration-150",
                            "focus:outline-none",
                            selectedCategory.id === category.id || hoveredCategory === category.id
                              ? "text-neutral-200"
                              : "text-neutral-400",
                          )}
                          whileTap={{ scale: 0.98 }}
                          variants={itemVariants}
                        >
                          <IconWrapper
                            icon={category.icon}
                            isHovered={hoveredCategory === category.id}
                            color={category.color}
                          />
                          {category.label}
                        </motion.button>
                      </React.Fragment>
                    ))}
                  </motion.div>
                </motion.div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </MotionConfig>
  )
}
