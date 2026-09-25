import json
binary_results = {}
current_binary = 'Unknown'
with open('test_results.txt', 'r', encoding='utf-16') as f:
    for line in f:
        line = line.strip()
        if line.startswith('Running unittests') or line.startswith('Running tests'):
            current_binary = line
            binary_results[current_binary] = []
        elif line.startswith('test ') and '...' in line:
            parts = line.split('...', 1)
            test_name = parts[0][5:].strip()
            status = parts[1].strip()
            if status.startswith('ok'): status = 'ok'
            elif status.startswith('FAILED'): status = 'FAILED'
            elif status.startswith('ignored'): status = 'ignored'
            binary_results.setdefault(current_binary, []).append({'name': test_name, 'status': status})

print(json.dumps(binary_results, indent=2))
