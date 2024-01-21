document.addEventListener('DOMContentLoaded', function () {
    var data = {
        protocol: [
            {
                name: "FTP",
                port: 2121
            },
            {
                name: "HTTP",
                port: 8080
            },
            {
                name: "TFTP",
                port: 6969
            }
        ]
    };

    // Get the template source
    var source = document.getElementById('entry-template').innerHTML;

    // Compile the template
    var template = Handlebars.compile(source);

    // Render the template with data
    var html = template(data);

    // Append the rendered HTML to the app div
    document.getElementById('app').innerHTML = html;
});
