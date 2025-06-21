## Memory overhead benchmark for key-value data structures:

Results interpretation:
1. Additional bytes overhead per element
2. Additional percantage of key size overhead per element
3. Additional percentage of value size overhead per element

```
Results for BTreeMap:
    > 4.61 + 78.4% K + 78.4% V
    +-0.13  +-0.9%  +-0.4%
Results for IndexMap:
    > 26.79 + 46.5% K + 46.5% V
    +-2.44  +-16.6%  +-6.6%
Results for HashMap:
    > 1.67 + 67.5% K + 67.5% V
    +-1.86  +-12.6%  +-5.1%
```
