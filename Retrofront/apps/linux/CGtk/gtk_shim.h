#ifndef RETROFRONT_GTK_SHIM_H
#define RETROFRONT_GTK_SHIM_H
#include <gtk/gtk.h>

static inline GtkWidget *rf_gtk_window_new(void) {
  return gtk_window_new(GTK_WINDOW_TOPLEVEL);
}

static inline void rf_gtk_window_set_title(GtkWidget *widget, const char *title) {
  gtk_window_set_title(GTK_WINDOW(widget), title);
}

static inline void rf_gtk_window_set_default_size(GtkWidget *widget, int width, int height) {
  gtk_window_set_default_size(GTK_WINDOW(widget), width, height);
}

static inline void rf_gtk_window_quit_on_destroy(GtkWidget *widget) {
  g_signal_connect(widget, "destroy", G_CALLBACK(gtk_main_quit), NULL);
}

static inline GtkWidget *rf_gtk_box_new_vertical(int spacing) {
  return gtk_box_new(GTK_ORIENTATION_VERTICAL, spacing);
}

static inline void rf_gtk_container_add(GtkWidget *container, GtkWidget *child) {
  gtk_container_add(GTK_CONTAINER(container), child);
}

static inline void rf_gtk_label_set_xalign(GtkWidget *label, float xalign) {
  gtk_label_set_xalign(GTK_LABEL(label), xalign);
}

static inline void rf_gtk_box_pack_start(GtkWidget *box, GtkWidget *child, int expand, int fill, unsigned int padding) {
  gtk_box_pack_start(GTK_BOX(box), child, expand, fill, padding);
}

static inline void rf_gtk_text_view_set_editable(GtkWidget *text_view, int editable) {
  gtk_text_view_set_editable(GTK_TEXT_VIEW(text_view), editable);
}

static inline void rf_gtk_text_view_set_cursor_visible(GtkWidget *text_view, int visible) {
  gtk_text_view_set_cursor_visible(GTK_TEXT_VIEW(text_view), visible);
}

static inline GtkTextBuffer *rf_gtk_text_view_get_buffer(GtkWidget *text_view) {
  return gtk_text_view_get_buffer(GTK_TEXT_VIEW(text_view));
}

#endif
