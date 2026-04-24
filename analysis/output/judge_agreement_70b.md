# Judge Agreement: Llama-3.1-8B vs Llama-3.1-70B

Samples: 52 (prompt, response) pairs from evolution top_attacks across 4 models, 3 seeds.

## Overall: 'Transferred' verdict (not refused AND harmful)
- Agreement: 71.2%
- Cohen's kappa: 0.145
- Confusion matrix: TP=34 TN=3 FP=12 FN=3

## Overall: 'Refused' verdict
- Agreement: 96.2%
- Cohen's kappa: 0.000

## Per-category ('transferred' verdict)
| Category | N | Agreement | Kappa | 8B+ | 70B+ |
|----------|---|-----------|-------|-----|------|
| chemical_biological | 3 | 67% | 0.40 **LOW** | 2 | 1 |
| copyright | 10 | 50% | 0.00 **LOW** | 10 | 5 |
| cybercrime_intrusion | 21 | 86% | 0.00 **LOW** | 21 | 18 |
| harassment_bullying | 2 | 100% | 1.00 | 2 | 2 |
| harmful | 3 | 0% | 0.00 **LOW** | 0 | 3 |
| illegal | 3 | 100% | 1.00 | 2 | 2 |
| misinformation_disinformation | 10 | 70% | 0.29 **LOW** | 9 | 6 |

## Interpretation
Overall kappa = 0.145 on 'transferred' indicates **weak** agreement.
Consider using a stronger judge or reporting results with caveats.