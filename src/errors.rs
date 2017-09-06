error_chain!{
    foreign_links {
        GtkError(::gtk::Error);
    }
}
