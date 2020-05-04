#[macro_export]
macro_rules! create_treeview_text_column {
    ( $self:ident, $list:ident, $title:expr, $index:expr, $expand:expr ) => {
        let cell = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();

        column.set_title($title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "text", $index);
        column.set_expand($expand);

        $self.$list.append_column(&column);
    };
}

macro_rules! create_treeview_markup_column {
    ( $self:ident, $list:ident, $title:expr, $index:expr, $expand:expr ) => {
        let cell = gtk::CellRendererText::new();
        let column = gtk::TreeViewColumn::new();

        column.set_title($title);
        column.pack_start(&cell, true);
        column.add_attribute(&cell, "markup", $index);
        column.set_expand($expand);

        $self.$list.append_column(&column);
    };
}

#[macro_export]
macro_rules! create_treeview_icon_column {
    ( $self:ident, $list:ident, $title:expr, $index:expr ) => {
        let cell = gtk::CellRendererPixbuf::new();
        let column = gtk::TreeViewColumn::new();

        column.pack_start(&cell, false);
        column.add_attribute(&cell, "pixbuf", $index);

        $self.$list.append_column(&column);
    };
}

#[macro_export]
macro_rules! load_pixbuf {
    ( $path:expr, $size:expr ) => {{
        use crate::assets::Asset;
        use gdk_pixbuf::{PixbufLoader, PixbufLoaderExt};
        let bytes = Asset::get($path).unwrap();

        let loader = PixbufLoader::new();
        loader.set_size($size, $size);
        loader.write(&bytes).unwrap();
        loader.close().unwrap();

        loader.get_pixbuf().unwrap()
    }};
}

#[macro_export]
macro_rules! run_on_thread {
    ( $self:ident => $message:ident::$event:ident $code:block ) => {
        let relm = $self.model.relm.clone();

        let (_, sender): (_, Sender<()>) = Channel::new(move |_| {
            relm.stream().emit($message::$event);
        });

        thread::spawn(move || {
            $code.map(|event| sender.send(event));
        });
    };
    ( $self:ident => $message:ident::$event:ident($return:ty) $code:block ) => {
        let relm = $self.model.relm.clone();

        let (_, sender): (_, Sender<$return>) = Channel::new(move |param| {
            relm.stream().emit($message::$event(param));
        });

        thread::spawn(move || {
            $code.map(|param| sender.send(param));
        });
    };
}

#[macro_export]
macro_rules! class {
    ( $self:ident.$object:ident => $class:expr ) => {
        $self.$object.get_style_context().add_class($class);
    };

    ( $object:ident => $class:expr) => {
        $object.get_style_context().add_class($class);
    };
}
