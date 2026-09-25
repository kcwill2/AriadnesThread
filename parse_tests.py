import subprocess
import re

def main():
    print("Running cargo test --workspace...")
    result = subprocess.run(['cargo', 'test', '--workspace'], capture_output=True, text=True)
    lines = result.stdout.split('\n')
    
    current_binary = None
    report = ["# Cargo Test Report\n"]
    
    for line in lines:
        line = line.strip()
        binary_match = re.match(r'Running (.*) \((.*)\)', line)
        if binary_match:
            current_binary = binary_match.group(1)
            report.append(f"\n## {current_binary}")
        elif line.startswith('test ') and ' ... ' in line:
            parts = line.split(' ... ')
            if len(parts) == 2:
                test_name = parts[0][5:] # remove 'test '
                status = parts[1]
                icon = "?" if status == "ok" else ("?" if status == "FAILED" else "??")
                report.append(f"- {icon} {test_name}: **{status}**")
        elif line.startswith('test result:'):
            report.append(f"\n**Summary**: {line}")

    with open('test_report.md', 'w', encoding='utf-8') as f:
        f.write('\n'.join(report))
    
    print("Report written to test_report.md")

if __name__ == '__main__':
    main()
