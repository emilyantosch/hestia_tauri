#!/usr/bin/env nu

# Development server wrapper with automatic Deno cleanup
# Usage: nu dev.nu

def kill_deno_processes [] {
    let deno_processes = (ps | where name =~ "deno" | get pid)
    
    if ($deno_processes | length) > 0 {
        print $"🧹 Cleaning up ($deno_processes | length) Deno process..."
        $deno_processes | each { |pid| 
            try {
                kill -9 $pid
                print $"  ✓ Killed Deno process (PID: ($pid))"
            } catch {
                print $"  ⚠ Failed to kill process (PID: ($pid)) - may already be dead"
            }
        }
    } else {
        print "✓ No Deno processes found to clean up"
    }
}

def main [] {
    print "🚀 Starting development server with Deno cleanup..."
    
    # Set up cleanup handler for Ctrl+C
    let cleanup_handler = {
        print "\n🛑 Received interrupt signal, cleaning up..."
        kill_deno_processes
        exit 0
    }
    
    try {
        # Run the development server
        deno task tauri dev
    } catch {
        print "💥 Development server stopped or failed"
    }
    
    # Always run cleanup when script ends
    print "🧹 Development server ended, running cleanup..."
    kill_deno_processes
    
    print "✅ Cleanup complete!"
}
