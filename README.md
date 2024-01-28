# Green Thumb Guide

![build](https://github.com/jelder/green_thumb_guide/actions/workflows/build.yml/badge.svg)

A GPT for improving my gardening skills.

### [Message Green Thumb Guide](https://chat.openai.com/g/g-YxvD5D5dO-green-thumb-guide)

## Configuration

> As an experienced horticulturist specializing in botany and plant care, my role is to offer detailed information on various aspects of gardening, including plant species identification, optimal growing conditions, pest management, and general plant maintenance. I aim to assist both novice and seasoned gardeners in understanding plant care nuances and achieving a thriving garden. My responses, rich in botanical knowledge, maintain a clear and informative tone. I avoid unrelated topics and clearly communicate the limitations of my advice when necessary, providing a confidence rating for reliability. Importantly, to enhance the accuracy and relevance of my guidance, I will request the user's ZIP code to give specific advice based on their USDA Hardiness Zone.

```yaml
openapi: 3.0.0
info:
  title: GreenThumb Hardiness Zone API
  version: 1.0.0
  description: API for retrieving USDA hardiness zone and minimum temperature information based on ZIP code.
servers:
  - url: https://greenthumb.jacobelder.com

paths:
  /hardiness_zone:
    get:
      summary: Get hardiness zone and minimum temperature by ZIP code
      operationId: getHardinessZone
      parameters:
        - name: q
          in: query
          required: true
          description: ZIP code to retrieve the hardiness zone and minimum temperature for.
          schema:
            type: string
            pattern: '^\d{5}$'
      responses:
        '200':
          description: Successful response with hardiness zone and temperature information.
          content:
            application/json:
              schema:
                type: object
                required:
                  - zone
                  - min_temp_f
                  - min_temp_c
                properties:
                  zone:
                    type: string
                    description: USDA Hardiness Zone.
                    example: '6b'
                  min_temp_f:
                    type: number
                    format: float
                    description: Lowest temperature in Fahrenheit observed in the last 30 years.
                    example: -5.0
                  min_temp_c:
                    type: number
                    format: float
                    description: Lowest temperature in Celsius observed in the last 30 years.
                    example: -20.56
        '400':
          description: Invalid ZIP code format.
        '404':
          description: Hardiness zone information not found for the given ZIP code.
        '500':
          description: Internal server error.
```