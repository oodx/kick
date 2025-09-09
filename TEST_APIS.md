# Test APIs for Kick Client

Collection of free APIs for testing the API client functionality:

## Basic JSON APIs
- **IP Address**: https://api.ipify.org/?format=json
  - Returns: `{"ip":"your.ip.address"}`
  - Good for: Basic GET requests, JSON parsing

- **Random Dog Images**: https://dog.ceo/api/breeds/image/random  
  - Returns: `{"message":"https://images.dog.ceo/breeds/.../image.jpg","status":"success"}`
  - Good for: Image URLs, download_file testing

- **Random Jokes**: https://official-joke-api.appspot.com/jokes/ten
  - Returns: Array of joke objects with setup/punchline
  - Good for: Larger JSON responses, array handling

## Additional Test APIs
- **Cat Facts**: https://catfact.ninja/fact
- **Random UUID**: https://httpbin.org/uuid  
- **User Agent Echo**: https://httpbin.org/user-agent
- **Headers Echo**: https://httpbin.org/headers
- **HTTP Status Codes**: https://httpbin.org/status/200 (or 404, 500, etc.)
- **Delay Testing**: https://httpbin.org/delay/2 (2 second delay)

## File Download Testing
- **Small JSON**: Any of the above APIs
- **Small Image**: Dog API returns image URLs for download_file testing
- **Text File**: https://httpbin.org/robots.txt

## Authentication Testing (Later)
- **Basic Auth**: https://httpbin.org/basic-auth/user/pass
- **Bearer Token**: https://httpbin.org/bearer

Perfect for testing GET, POST, downloads, error handling, and timeouts!