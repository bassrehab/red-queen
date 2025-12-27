# Security Policy

## Purpose

Red Queen is a security research tool designed to improve AI safety by discovering vulnerabilities. This document outlines responsible use and disclosure practices.

## Responsible Use

### Intended Use Cases

- Security research on AI systems you own or have permission to test
- Improving defensive capabilities of AI systems
- Academic research with appropriate IRB approval
- Red team exercises with explicit authorization
- Contributing to AI safety research

### Prohibited Uses

- Attacking production systems without authorization
- Generating harmful content for malicious purposes
- Circumventing safety measures for harm
- Selling or distributing discovered exploits
- Using discovered vulnerabilities for profit or harm

## Vulnerability Disclosure

### If You Discover a Vulnerability Using Red Queen

1. **Do NOT** publish the specific exploit publicly
2. **Do NOT** use it against production systems
3. **DO** report it to the affected party through their security channels
4. **DO** follow coordinated disclosure timelines (typically 90 days)

### Reporting Process

For vulnerabilities in major AI systems:

| Provider | Contact |
|----------|---------|
| OpenAI | security@openai.com |
| Anthropic | security@anthropic.com |
| Google | https://bughunters.google.com |
| Meta | https://www.facebook.com/whitehat |

### What to Include in Reports

1. Description of the vulnerability
2. Steps to reproduce (sanitized if necessary)
3. Potential impact assessment
4. Suggested mitigations (if any)
5. Your contact information

## Security in Red Queen

### Reporting Security Issues in Red Queen

If you discover a security vulnerability in Red Queen itself:

1. **Do NOT** open a public issue
2. Email: [security contact - to be added]
3. Include: description, reproduction steps, impact assessment

We will:
- Acknowledge receipt within 48 hours
- Provide an initial assessment within 7 days
- Work with you on a fix timeline
- Credit you in the security advisory (if desired)

### Security Design

Red Queen follows these security principles:

1. **No credential storage** - API keys are environment variables only
2. **Audit logging** - All attacks are logged for accountability
3. **Rate limiting** - Built-in limits prevent abuse
4. **No phone-home** - No telemetry sent without explicit consent

## Legal Considerations

### Authorization

Always ensure you have explicit written authorization before testing any system. This includes:
- Your own systems
- Systems you have permission to test
- Bug bounty programs with clear scope
- Academic research with IRB approval

### Compliance

Be aware of relevant laws and regulations:
- Computer Fraud and Abuse Act (CFAA) - US
- Computer Misuse Act - UK
- GDPR - EU (for data handling)
- Local computer crime laws

## Research Ethics

### Academic Research

If using Red Queen for academic research:
1. Obtain IRB/ethics board approval
2. Follow institutional guidelines
3. Consider dual-use implications
4. Publish responsibly (see below)

### Responsible Publication

When publishing research using Red Queen:
1. Coordinate with affected parties before publication
2. Delay disclosure of specific exploits until patched
3. Focus on methodology, not specific exploit strings
4. Include defensive recommendations
5. Consider the broader implications of your findings

## Contact

For security-related questions:
- GitHub Security Advisories (preferred)
- Email: [to be added]

For general questions:
- GitHub Issues
- GitHub Discussions
