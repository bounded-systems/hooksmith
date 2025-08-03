#!/bin/bash
# Script to clean up log files and manage hooksmith-events.jsonl

set -e

LOG_FILE="hooksmith-events.jsonl"
MAX_SIZE_MB=100  # Maximum file size in MB before rotation
MAX_AGE_DAYS=7   # Maximum age in days before deletion
BACKUP_DIR="logs/backup"

echo "🧹 Cleaning up Hooksmith log files..."

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Function to get file size in MB
get_file_size_mb() {
    if [ -f "$1" ]; then
        du -m "$1" | cut -f1
    else
        echo "0"
    fi
}

# Function to get file age in days
get_file_age_days() {
    if [ -f "$1" ]; then
        echo $(( ($(date +%s) - $(stat -f %m "$1")) / 86400 ))
    else
        echo "999"
    fi
}

# Check if log file exists
if [ ! -f "$LOG_FILE" ]; then
    echo "📝 No log file found at $LOG_FILE"
    exit 0
fi

# Get current file size and age
current_size=$(get_file_size_mb "$LOG_FILE")
current_age=$(get_file_age_days "$LOG_FILE")

echo "📊 Current log file stats:"
echo "   Size: ${current_size}MB"
echo "   Age: ${current_age} days"

# Rotate if file is too large
if [ "$current_size" -gt "$MAX_SIZE_MB" ]; then
    echo "📦 Log file is large (${current_size}MB > ${MAX_SIZE_MB}MB), rotating..."
    timestamp=$(date +"%Y%m%d_%H%M%S")
    backup_file="$BACKUP_DIR/hooksmith-events_${timestamp}.jsonl"
    
    mv "$LOG_FILE" "$backup_file"
    echo "✅ Rotated to $backup_file"
    
    # Compress old backup files
    find "$BACKUP_DIR" -name "hooksmith-events_*.jsonl" -mtime +1 -exec gzip {} \;
    echo "🗜️  Compressed backup files older than 1 day"
fi

# Clean up old backup files
echo "🗑️  Cleaning up old backup files..."
find "$BACKUP_DIR" -name "hooksmith-events_*.jsonl.gz" -mtime +$MAX_AGE_DAYS -delete
find "$BACKUP_DIR" -name "hooksmith-events_*.jsonl" -mtime +$MAX_AGE_DAYS -delete

# Show cleanup summary
backup_count=$(find "$BACKUP_DIR" -name "hooksmith-events_*" | wc -l | tr -d ' ')
echo "📋 Cleanup summary:"
echo "   Backup files: $backup_count"
echo "   Max file size: ${MAX_SIZE_MB}MB"
echo "   Max age: ${MAX_AGE_DAYS} days"

# Show disk usage
if [ -d "$BACKUP_DIR" ]; then
    backup_size=$(du -sh "$BACKUP_DIR" | cut -f1)
    echo "   Total backup size: $backup_size"
fi

echo "✅ Log cleanup completed!" 
