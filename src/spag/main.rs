extern crate docopt;
use docopt::Docopt;

docopt!(Args derive Debug, "
Usage: spag request [show] <file> [(-H <header>)...]
       spag (get|post|put|patch|delete) <resource> [(-H <header>)...]
       spag env set (<key> <val>)...
       spag env show [<environment>]
       spag history
       spag history show <index>

Options:
    -h, --help      Show this message
    -H, --header    Supply a header

Arguments:
    <resource>      The path of an api resource, like /v2/things
    <header>        An http header, like 'Content-type: application/json'
    <environment>   The name of an environment, like 'default'
    <index>         An index, starting at zero
");

pub fn main() {
    let args: Args = Args::docopt().decode().unwrap_or_else(|e| e.exit());
    println!("{:?}", args);

    if args.cmd_request {
        spag_request(&args);
    } else if args.cmd_history {
        spag_history(&args);
    } else if args.cmd_env {
        spag_env(&args);
    } else if args.cmd_get || args.cmd_post || args.cmd_put || args.cmd_patch || args.cmd_delete {
        spag_method(&args);
    }
}

fn spag_env(args: &Args) {
    println!("called spag env");
}

fn spag_history(args: &Args) {
    println!("called spag history");
}

fn spag_request(args: &Args) {
    println!("called spag request");
}

fn spag_method(args: &Args) {
    println!("called spag method");
}

