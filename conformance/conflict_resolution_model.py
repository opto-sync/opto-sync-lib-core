#!/usr/bin/env python3
from itertools import product

def resolve(a,b):
    # tuple(revision, tombstone, replica_id); total order makes resolution deterministic/commutative
    return max((a,b), key=lambda x:(x[0],x[1],x[2]))
def main():
    states=list(product(range(3),(False,True),range(2)))
    checked=0
    for a in states:
        for b in states:
            r1=resolve(a,b); r2=resolve(b,a)
            assert r1==r2,'conflict resolution not commutative'
            assert r1[0]>=min(a[0],b[0])
            if a[0]==b[0] and (a[1] or b[1]): assert r1[1],'equal-revision tombstone lost'
            checked+=1
    print(f'conflict resolution model: {len(states)} states, {checked} pairs')
if __name__=='__main__': main()
