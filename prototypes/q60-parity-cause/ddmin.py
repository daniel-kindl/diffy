import subprocess, sys, os
S, H, L, R = sys.argv[1:5]
work = os.path.join(S, 'dd')
def fails(left, right):
    lp, rp = os.path.join(work, 'l.txt'), os.path.join(work, 'r.txt')
    open(lp, 'wb').write(b''.join(left)); open(rp, 'wb').write(b''.join(right))
    tsv = os.path.join(work, 'p.tsv'); open(tsv, 'w').write(f'x\t{lp}\t{rp}\n')
    out = subprocess.run([H, tsv, os.path.join(work, 'o.json')], capture_output=True, text=True).stdout
    for line in out.splitlines():
        if line.startswith('--indent-heuristic') and 'GixIndentHeuristic' in line:
            return 'fail      1' in line
    raise SystemExit(out)
def lines(p): return open(p, 'rb').read().splitlines(keepends=True)
left, right = lines(L), lines(R)
assert fails(left, right)
# Alternate: shrink each side with chunked removal until no chunk can go.
changed = True
while changed:
    changed = False
    for side in (0, 1):
        seq = [left, right][side]
        n = max(1, len(seq) // 2)
        while n >= 1:
            i = 0
            while i < len(seq):
                cand = seq[:i] + seq[i+n:]
                t = (cand, right) if side == 0 else (left, cand)
                if fails(*t):
                    seq = cand; changed = True
                    if side == 0: left = seq
                    else: right = seq
                else:
                    i += n
            n //= 2
    print(len(left), len(right), flush=True)
open(os.path.join(work, 'min.left'), 'wb').write(b''.join(left))
open(os.path.join(work, 'min.right'), 'wb').write(b''.join(right))
