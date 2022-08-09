# imageserver
A bare bones image server

This project is considered basically done for its intended purpose

The apis are as follows

## GET
Gets the specific image
https://localhost/v1/images/image.png

Gets an embeded url through the image server and gives it to the client so only the server ip is revealed and not the client ip
https://localhost/v1/embed?url=https://somescetchywebsite.com/image.png

## POST
https://localhost/v1/images

